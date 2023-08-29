enum Forward<T> {
    WaitingForReceive(ReceiveFuture<T>, Option<Sender<T>>),
    WaitingForSend(SendFuture<T>, Option<Receiver<T>>),
}

impl<T> Future for Forward<T> {}
