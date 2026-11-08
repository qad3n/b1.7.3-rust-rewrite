pub trait IProgressUpdate
{
    fn progressStartNoAbort(&self, text: &str);
    fn displayLoadingString(&self, text: &str);
    fn setLoadingProgress(&self, progress: i32);
}
