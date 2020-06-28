using System.Collections.Generic;
using CheetahRelay.Runtime.LowLevel.Listener;

namespace CheetahRelay.Tests.Runtime
{
    public class RelayClientListenerStub : IRelayClientListener
    {
        public List<string> commands = new List<string>();

        public void OnConnected()
        {
            commands.Add("OnConnected");
        }

        public void OnDisconnect()
        {
            commands.Add("OnDisconnect");
        }
    }
}