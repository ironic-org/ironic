use ironic::prelude::*;

#[controller("/my-ctrl")]
#[derive(Injectable)]
pub struct MyCtrlController;

#[routes]
impl MyCtrlController {
    #[get("/")]
    #[api(summary = "List my-ctrl", tag = "MyCtrl")]
    #[resp(200, "OK")]
    #[allow(clippy::unused_async)]
    async fn list(&self) -> Result<&'static str, HttpError> {
        Ok("my-ctrl controller")
    }
}
