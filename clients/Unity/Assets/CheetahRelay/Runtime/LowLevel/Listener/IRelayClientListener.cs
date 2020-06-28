namespace CheetahRelay.Runtime.LowLevel.Listener
{
    public interface IRelayClientListener
    {
        void OnConnected();

        void OnDisconnect();
    }
}