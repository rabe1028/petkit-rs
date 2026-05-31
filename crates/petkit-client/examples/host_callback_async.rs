use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

use petkit_client::host_callback::HostCallbackTransport;
use petkit_client::AsyncPetkitClient;
use petkit_protocol::{BaseUrl, RequestSpec, ResponseParts};
use petkit_types::{ClientContext, ClientProfile};

async fn host_send(
    requests: Rc<RefCell<Vec<String>>>,
    request: RequestSpec,
) -> Result<ResponseParts, Infallible> {
    requests.borrow_mut().push(request.path);
    Ok(ResponseParts::new(
        200,
        vec![],
        br#"{"result":[]}"#.to_vec(),
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let transport = HostCallbackTransport::from_fn({
        let requests = Rc::clone(&requests);
        move |request| host_send(Rc::clone(&requests), request)
    });
    let context = ClientContext::new(ClientProfile::default(), "UTC", "0");
    let client = AsyncPetkitClient::with_session(
        context,
        BaseUrl::Regional("https://api.petkt.com/latest/".into()),
        "session-id",
        transport,
    );

    let devices = futures::executor::block_on(client.device_list())?;
    assert!(devices.is_empty());
    assert_eq!(requests.borrow().as_slice(), &["group/family/list"]);
    Ok(())
}
