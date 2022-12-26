using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;

namespace Games.Cheetah.Client
{
    public static class MockNetworkClientFactory
    {
        public static (INetworkClientMock clientMock, NetworkClient client) Create(CodecRegistry codecRegistry)
        {
            var mock = new FFIMock(codecRegistry);
            var client = new NetworkClient(mock, "host", 555, 55, 55, new byte[] { }, codecRegistry);
            return (mock, client);
        }
    }
}