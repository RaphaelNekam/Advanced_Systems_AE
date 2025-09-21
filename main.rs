use std::thread;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use crate::mpsc::Receiver;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::TcpStream;
use std::time::Duration;
use std::io::Write;
use std::io::Read;

// Structure to keep thread, channel, and IP address together
struct ConnectionAttempt {
    thread: thread::JoinHandle<()>,
    tx: Sender<SocketAddr>,
    address: SocketAddr
}

// Takes a hostname (e.g. www.google.com) and returns all found IP addresses
fn dnslookup(hostname: &String) -> Vec<SocketAddr> {
    let lookup_addresses = (hostname.clone(), 80).to_socket_addrs();
    let addresses: Vec<SocketAddr> = match lookup_addresses {
        Ok(addresses) => addresses.collect(),
        Err(_) => {
            println!("Error for address lookup of {} occurred.", hostname);
            Vec::new()
        }
    };

    addresses
}

// takes a stream & corresponding hostname and establishes a HTTP request
// Prints the response from the host
fn handle_stream(mut stream: &TcpStream, hostname: String) {

    // Send request
    let request = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", hostname);
    let _ = stream.write(request.as_bytes());

    // Set timeout
    stream.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();


    // Read response
    let mut buffer = [0; 1024]; // Fixed-size buffer
    loop {
        let bytes_read = stream.read(&mut buffer).expect("Failed to read from server");

        if bytes_read == 0 {
            //println!("\n End of response reached, connection closed by server.");
            break; // Server closed the connection
        }

        // print response
        println!("{}", String::from_utf8_lossy(&buffer));
    }
}

fn main() {

    // counts the number of all connection attempt threads created
    let mut connection_attempt_counter = 0;

    // arguments obtained
    let args: Vec<String> = env::args().collect();

    let hostname = args[1].clone();
    let addresses = dnslookup(&hostname);

    // channel for communication between connection attempt threads (ca) and connected client threads (cc)
    let (ca_cc_tx, ca_cc_rx): (Sender<TcpStream>, Receiver<TcpStream>) = mpsc::channel();

    // structure to hold all connection attempt structs
    let mut connection_attempts = Vec::new();

    for address in addresses {

        // channel for communication between main thread (mt) and connection attempt (ca) threads
        let (mt_ca_tx, mt_ca_rx) = mpsc::channel::<SocketAddr>();

        // clone to allow use for individual connection attempt (ca) thread
        let ca_cc_tx_clone = ca_cc_tx.clone();

        // create a single connection attempt (ca) thread for each address
        let ca_thread = thread::spawn( move || {

                // receive address to attempt from main thread
                let addr: SocketAddr = match mt_ca_rx.recv() {
                    Ok(addr) => addr,
                    Err(e) => {
                        println!("Failed to receive address: {:?}", e);
                        return;
                    }
                };
                let stream = match TcpStream::connect(addr) {
                    Ok(stream) => stream,
                    Err(e) => {
                        println!("Failed to connect to {:?}: {:?}", addr, e); // Print the error
                        return;
                    }
                };

                // if connection established, share TcpStream with connected client thread
                let _ = ca_cc_tx_clone.send(stream);
        });

        // add this thread to the connection attempt structure
        connection_attempts.push(ConnectionAttempt {
            thread: ca_thread,
            tx: mt_ca_tx,
            address
        });

        connection_attempt_counter += 1;
    }

    // create the connected client thread
    let connected_client_thread = thread::spawn(move || {

        let mut first_received = false;

        // while at least one connection attempt thread is still running
        while connection_attempt_counter > 0 {

            // receive TcpStream from connection attempt threads
            let connected_stream: TcpStream = match ca_cc_rx.recv() {
                Ok(connected_stream) => connected_stream,
                Err(error) => {
                    println!("Failed to receive stream: {:?}", error);
                    return;
                }
            };      

            // if this is the first TcpStream received -> handle stream to obtain response
            if !first_received {
                first_received = true;
                println!("{:?}", connected_stream.peer_addr().unwrap());
                handle_stream(&connected_stream, hostname.clone());
            }
            
            // close TcpStream
            connection_attempt_counter -= 1;
            // println!("DROP:  {:?}", connected_stream.peer_addr());
            drop(connected_stream);       
        }
    });

    for connection_attempt in connection_attempts {
        // send address to each connection attempt thread
        let _ = connection_attempt.tx.send(connection_attempt.address);

        // wait for all connection attempt threads to join
        let _ = connection_attempt.thread.join();
    }

    // wait for connected client thread to join
    let _ = connected_client_thread.join();
}
