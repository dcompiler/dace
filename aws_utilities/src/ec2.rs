use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;

fn connectToEC2() {
    // Use the IP address or hostname of your EC2 instance
    let tcp = TcpStream::connect("ec2-xx-xxx-xxx-xx.us-west-2.compute.amazonaws.com:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // Use your private key for the EC2 instance
    sess.userauth_pubkey_file("ec2-user", None, "/path/to/your/key.pem", None).unwrap();

    assert!(sess.authenticated());

    let mut channel = sess.channel_session().unwrap();

    // Navigate to the directory and run git commit
    channel.exec("cd /path/to/repo && git add file_to_commit && git commit -m 'Commit message'").unwrap();

    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();

    println!("{}", s);

    // Get the latest commit hash
    channel.exec("cd /path/to/repo && git rev-parse HEAD").unwrap();

    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();

    println!("{}", s);

    channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
}
