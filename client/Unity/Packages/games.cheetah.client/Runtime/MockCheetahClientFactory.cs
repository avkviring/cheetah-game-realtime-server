using System;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.ServerAPI.Mock;

namespace Games.Cheetah.Client
{
    public static class MockCheetahClientFactory
    {
        public static Tuple<ICheetahClientMock, CheetahClient> Create(CodecRegistry codecRegistry)
        {
            {
                var serverMock = new CheetahClientMock(codecRegistry);
                var cheetahClient = new CheetahClient(serverMock, "", 0, 0, 0, new byte[] { }, codecRegistry);
                return new Tuple<ICheetahClientMock, CheetahClient>(serverMock, cheetahClient);
            }
        }
    }
}