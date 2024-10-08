use std::{
    io::{self, BufReader},
    net::{SocketAddr, TcpStream},
};

use compression::{brotli, gzip, zlib};
use dns::DNSError;
use error_derive::Error;
use url::{Host, URL};

use crate::{https, response::Response, Header, Headers, StatusCode};

const USER_AGENT: &str = "Stormlicht";
pub(crate) const HTTP_NEWLINE: &str = "\r\n";

const MAX_REDIRECTS: usize = 32;

#[derive(Debug, Error)]
pub enum HTTPError {
    #[msg = "invalid response"]
    InvalidResponse,

    #[msg = "status code indicates error"]
    Status(StatusCode),

    #[msg = "io error"]
    IO(io::Error),

    #[msg = "failed to resolve host"]
    DNS(DNSError),

    #[msg = "gzip decompression failed"]
    Gzip(gzip::Error),

    #[msg = "brotli decompression failed"]
    Brotli(brotli::Error),

    #[msg = "zlib decompression failed"]
    Zlib(zlib::Error),

    #[msg = "tls communication failed"]
    Tls(rustls::Error),

    #[msg = "too many redirections"]
    RedirectLoop,

    #[msg = "redirect to non-http url"]
    NonHTTPRedirect,

    #[msg = "request to non-http url"]
    NonHTTPURl,
}

#[derive(Clone, Debug)]
pub struct Context {
    /// The number of times we were redirected while completing
    /// the original request
    pub num_redirections: usize,

    /// The [URL] that is currently being loaded
    pub url: URL,

    pub proxy: Option<SocketAddr>,
}

/// HTTP Request Method
///
/// Refer to the relevant specifications for more information:
/// * <https://tools.ietf.org/html/rfc7231#section-4.1>
/// * <https://datatracker.ietf.org/doc/html/rfc5789>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/CONNECT>
    Connect,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/DELETE>
    Delete,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/GET>
    Get,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/HEAD>
    Head,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS>
    Options,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/PATCH>
    Patch,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/POST>
    Post,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/PUT>
    Put,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/TRACE>
    Trace,
}

#[derive(Clone, Debug)]
pub struct Request {
    method: Method,
    headers: Headers,
    context: Context,
}

impl Context {
    #[must_use]
    pub const fn new(url: URL) -> Self {
        Self {
            num_redirections: 0,
            url,
            proxy: None,
        }
    }

    pub fn set_proxy(&mut self, proxy: SocketAddr) {
        self.proxy = Some(proxy);
    }
}

impl Method {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Connect => "CONNECT",
            Self::Delete => "DELETE",
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Patch => "PATCH",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Trace => "TRACE",
        }
    }
}

impl Request {
    /// Create a `GET` request for the specified URL
    ///
    /// # Panics
    /// This function panics if the url scheme is not `http`
    /// or the url does not have a `host`.
    #[must_use]
    pub fn get(url: &URL) -> Self {
        assert!(
            matches!(url.scheme().as_str(), "http" | "https"),
            "URL is not http(s)"
        );

        let mut headers = Headers::with_capacity(3);
        headers.set(Header::USER_AGENT, USER_AGENT.to_string());
        headers.set(Header::ACCEPT, "*/*".to_string());
        headers.set(
            Header::ACCEPT_ENCODING,
            "gzip, brotli, deflate, identity".to_string(),
        );
        headers.set(
            Header::HOST,
            url.host().expect("URL does not have a host").to_string(),
        );

        Self {
            method: Method::Get,
            headers,
            context: Context::new(url.clone()),
        }
    }

    pub fn set_proxy(&mut self, proxy: SocketAddr) {
        self.context.set_proxy(proxy);
    }

    #[must_use]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    #[must_use]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Serialize the request to the given [Writer](io::Write)
    fn write_to<W>(&self, mut writer: W) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        // Send request header
        let path = if self.context.proxy.is_none() {
            self.context.url.path()
        } else {
            self.context.url.serialize(url::ExcludeFragment::Yes)
        };

        write!(
            writer,
            "{method} {path} HTTP/1.1{HTTP_NEWLINE}",
            method = self.method.as_str(),
            path = path,
        )?;

        // Send headers
        for (header, value) in self.headers.iter() {
            write!(writer, "{}: {value}{HTTP_NEWLINE}", header.as_str())?;
        }

        // Finish request with an extra newline
        write!(writer, "{HTTP_NEWLINE}")?;

        writer.flush()?;
        Ok(())
    }

    pub fn send(&mut self) -> Result<Response, HTTPError> {
        if let Some(proxy) = self.context.proxy {
            log::info!("Proxying http connection via {proxy}");
            let stream = TcpStream::connect(proxy)?;
            return self.send_on_stream(stream);
        }

        // Establish a connection with the host
        let host = self.context.url.host().expect("url does not have a host");
        let port = self.context.url.port();

        match self.context.url.scheme().as_str() {
            "http" => {
                // Resolve the hostname
                let ip = match &host {
                    Host::Domain(host) | Host::OpaqueHost(host) => dns::Domain::new(host.as_str())
                        .lookup()
                        .map_err(HTTPError::DNS)?,
                    Host::Ip(_ip) => todo!(),
                    Host::EmptyHost => todo!(),
                };

                let stream = TcpStream::connect(SocketAddr::new(ip, port.unwrap_or(80)))?;
                self.send_on_stream(stream)
            },
            "https" => {
                let stream = match host {
                    Host::Domain(host) | Host::OpaqueHost(host) => {
                        https::establish_connection(host.to_string(), port)?
                    },
                    _ => todo!(),
                };
                self.send_on_stream(stream)
            },
            _ => Err(HTTPError::NonHTTPURl),
        }
    }

    fn send_on_stream<S: io::Read + io::Write>(
        &mut self,
        mut stream: S,
    ) -> Result<Response, HTTPError> {
        // Send our request
        self.write_to(&mut stream)?;

        // Parse the response
        let mut reader = BufReader::new(stream);
        let response = Response::receive(&mut reader, self.context.clone())?;

        if response.status().is_error() {
            log::warn!("HTTP Request failed: {:?}", response.status());
            return Err(HTTPError::Status(response.status()));
        }

        if response.status().is_redirection() {
            if let Some(relocation) =
                response
                    .headers()
                    .get(Header::LOCATION)
                    .and_then(|location| {
                        URL::parse_with_base(location, Some(&self.context.url), None).ok()
                    })
            {
                log::info!(
                    "{current_url} redirects to {redirect_url} ({status_code:?})",
                    current_url = self.context.url.serialize(url::ExcludeFragment::No),
                    redirect_url = relocation.serialize(url::ExcludeFragment::No),
                    status_code = response.status()
                );

                if !matches!(relocation.scheme().as_str(), "http" | "https") {
                    log::error!(
                        "Cannot load non-http redirect url: {redirect_url}",
                        redirect_url = relocation.serialize(url::ExcludeFragment::Yes)
                    );
                    return Err(HTTPError::NonHTTPRedirect);
                }

                self.context.num_redirections += 1;

                if self.context.num_redirections >= MAX_REDIRECTS {
                    log::warn!("Too many HTTP redirections ({MAX_REDIRECTS}), stopping");
                    return Err(HTTPError::RedirectLoop);
                }

                self.headers.set(
                    Header::HOST,
                    relocation
                        .host()
                        .expect("relocation url does not have a host")
                        .to_string(),
                );
                self.context.url = relocation;
                return self.send();
            } else {
                log::warn!("HTTP response indicates redirection, but no new URL could be found");
            }
        }

        Ok(response)
    }
}
