use ssh2::{Session, Sftp};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::Path,
};

/// Creates a new SFTP connection.
pub fn connect<A>(addr: A, user: &str, pwd: &str) -> std::io::Result<Sftp>
where
    A: ToSocketAddrs,
{
    let tcp = TcpStream::connect(addr)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(user, pwd)?;

    sess.sftp().map_err(std::io::Error::from)
}

/// Upload a file tho a remote path.
pub fn upload<F, R>(conn: &Sftp, file: F, remote: R) -> std::io::Result<()>
where
    F: AsRef<Path>,
    R: AsRef<Path>,
{
    let mut rdr = BufReader::new(File::open(file)?);
    let mut buffer = Vec::new();
    rdr.read_to_end(&mut buffer)?;

    conn.create(remote.as_ref())?.write_all(&buffer)?;
    Ok(())
}
