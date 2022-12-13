using System;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock
{
    public class ClientAPIMock : IClientServerAPI
    {
        public event Action OnReceive;

        public byte CreateClient(string serverAddress, ushort memberId, ulong roomId, ref CheetahBuffer userPrivateKey, ulong startFrameId,
            out ushort clientId)
        {
            clientId = 0;
            return 0;
        }

        public byte GetConnectionStatus(ushort clientId, out CheetahClientConnectionStatus status)
        {
            status = CheetahClientConnectionStatus.Connected;
            return 0;
        }

        public byte GetStatistics(ushort clientId, out CheetahClientStatistics clientStatistics)
        {
            clientStatistics = new CheetahClientStatistics();
            return 0;
        }

        public byte Receive(ushort clientId)
        {
            OnReceive.Invoke();
            return 0;
        }

        public byte DestroyClient(ushort clientId)
        {
            return 0;
        }

        public byte AttachToRoom(ushort clientId)
        {
            return 0;
        }

        public byte DetachFromRoom(ushort clientId)
        {
            return 0;
        }

        public byte SetChannelType(ushort clientId, ChannelType channelType, byte group)
        {
            return 0;
        }

        public byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion)
        {
            return 0;
        }

        public byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs)
        {
            return 0;
        }

        public byte ResetEmulation(ushort clientId)
        {
            return 0;
        }

        public void GetLastErrorMsg(ref CheetahBuffer buffer)
        {
        }

        public byte GetServerTime(ushort clientId, out ulong time)
        {
            time = 0;
            return 0;
        }
    }
}