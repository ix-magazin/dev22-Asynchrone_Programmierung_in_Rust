/////////////////////////////////////////////////////////
// Listing 1: Lottozahlen als Grundlage fuer Threading //
/////////////////////////////////////////////////////////

fn main() {
  let lottos = Mutex::new(Vec::<Lotto>::new()); // 1
  let lottos = Arc::new(lottos); // 2
  let mut handles = Vec::new();
  let pairs = [(6, 45), (5, 50), (2, 12)];

  for (take, from) in pairs {
    let lottos = Arc::clone(&lottos); // 3
    let handle = thread::spawn(move || { // 4
      let lotto = Lotto::new(take, from);
      lottos.lock().unwrap().push(lotto); // 5
    });

    handles.push(handle);
  }

  for handle in handles {
    handle.join().unwrap(); // 6
  }

  for lotto in lottos.lock().unwrap().iter() {
    println!("{:?}", lotto);
  }
}


////////////////////////////////////////////////////////////
// Listing 2: HTTP-Request ohne asynchrone Programmierung //
////////////////////////////////////////////////////////////

fn request(host: &str, port: u16, path: &str)-> 
  std::io::Result<String> {
    let mut socket = 
      net::TcpStream::connect((host, port))?; // Hier

    let request = 
      format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", 
              path, host);
    socket.write_all(request.as_bytes())?;    // Hier
    socket.shutdown(net::Shutdown::Write)?;

    let mut response = String::new();
    socket.read_to_string(&mut response)?;    // Hier

    Ok(response)
}


///////////////////////////////////////////////
// Listing 3: Als Future abstrahiertes Delay //
///////////////////////////////////////////////

struct Delay {
  when: Instant,
}

impl Future for Delay {
  type Output = &'static str;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
    -> Poll<&'static str> {
    if Instant::now() >= self.when {
      Poll::Ready("done")
    } else {
      let waker = cx.waker().clone();
      let when = self.when;

      thread::spawn(move || {
        let now = Instant::now();

        if now < when {
          thread::sleep(when - now);
        }
        waker.wake();
      });
      Poll::Pending
    }
  }
}


//////////////////////////////////////////////
// Listing 4: Umsetzung mit dem Tokio-Stack //
//////////////////////////////////////////////

#[tokio::main]
async fn main() {
  let when = Instant::now() + Duration::from_millis(10);
  let future = Delay { when };

  let out = future.await;
  assert_eq!(out, "done");
}


//////////////////////////////////
// Listing 5: Generierte Future //
//////////////////////////////////

enum MainFuture {
  // Initialized, never polled
  State0,
  // Waiting on `Delay`, i.e. the `future.await` line.
  State1(Delay),
  // The future has completed.
  Terminated,
}


///////////////////////////////////////////////
// Listing 6: Implementierung der MainFuture //
///////////////////////////////////////////////

impl Future for MainFuture {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
    -> Poll<()>
  {
    use MainFuture::*;

    loop {
      match *self {
        State0 => {
          let when = Instant::now() +
              Duration::from_millis(10);
          let future = Delay { when };
          *self = State1(future);
        }
        State1(ref mut my_future) => {
          match Pin::new(my_future).poll(cx) {
            Poll::Ready(out) => {
              assert_eq!(out, "done");
              *self = Terminated;
              return Poll::Ready(());
            }
            Poll::Pending => {
              return Poll::Pending;
            }
          }
        }
        Terminated => {
          panic!("future polled after completion")
        }
      }
    }
  }
}


/////////////////////////////////////////
// Listing 7: Asynchroner HTTP-Request //
/////////////////////////////////////////

async fn request(host: &str, port: u16, path: &str) -> 
  std::io::Result<String> {
    // from Tokio (1):
    let mut socket =  net::TcpStream::connect((host, port)).await?;

    let request = 
      format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes()).await?; // from Tokio (2)
    socket.shutdown(net::Shutdown::Write)?;

    let mut response = String::new();
    socket.read_to_string(&mut response).await?; // from Tokio (3)

    Ok(response)
}


//////////////////////////////////////////////
// Listing 8: Geradliniger asynchroner Code //
//////////////////////////////////////////////

async fn main() {
  for await? issue in crabbycat::issues("https://github.com/rust-lang/rust") {
    if meets_criteria(&issue) {
      println!("{issue:?}");
    }
  }
}


/////////////////////////////////////////////////////////
// Listing 9: Traits in der asynchronen Programmierung //
/////////////////////////////////////////////////////////

trait IssueProvider {
    async fn issues(&mut self, url: &str)
        -> impl AsyncIterator<Item = Result<Issue, Err>>;
}

#[derive(Debug)]
struct Issue {
    number: usize,
    header: String,
    assignee: String,
}

fn process_issues(provider: &mut dyn IssueProvider) {
    for await? issue in provider.issues("https://github.com/rust-lang/rust") {
        if meets_criteria(&issue) {
            println!("{issue:?}");
        }
    }
}
