using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class ClientServerAPI : IClientServerAPI
    {
        public byte CreateClient(
            string serverAddress,
            ushort memberId,
            ulong roomId,
            ref CheetahBuffer userPrivateKey,
            ulong startFrameId,
            out ushort clientId
        )
        {
            return ClientFFI.CreateClient(serverAddress, memberId, roomId, ref userPrivateKey, startFrameId, out clientId);
        }

        public byte GetConnectionStatus(ushort clientId, out CheetahClientConnectionStatus status)
        {
            return ClientFFI.GetConnectionStatus(clientId, out status);
        }


        public byte GetStatistics(ushort clientId, out CheetahClientStatistics clientStatistics)
        {
            return ClientFFI.GetStatistics(clientId, out clientStatistics);
        }


        public byte Receive(ushort clientId)
        {
            return ClientFFI.Receive(clientId);
        }


        public byte DestroyClient(ushort clientId)
        {
            return ClientFFI.DestroyClient(clientId);
        }


        public byte AttachToRoom(ushort clientId)
        {
            return ClientFFI.AttachToRoom(clientId);
        }


        public byte DetachFromRoom(ushort clientId)
        {
            return ClientFFI.DetachFromRoom(clientId);
        }


        public byte SetChannelType(ushort clientId, ChannelType channelType, byte group)
        {
            return ClientFFI.SetChannelType(clientId, channelType, group);
        }


        public byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion)
        {
            return ClientFFI.SetRttEmulation(clientId, rttInMs, rttDispersion);
        }


        public byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs)
        {
            return ClientFFI.SetDropEmulation(clientId, dropProbability, dropTimeInMs);
        }


        public byte ResetEmulation(ushort clientId)
        {
            return ClientFFI.ResetEmulation(clientId);
        }


        public void GetLastErrorMsg(ref CheetahBuffer buffer)
        {
            ClientFFI.GetLastErrorMsg(ref buffer);
        }

        public byte GetServerTime(ushort clientId, out ulong time)
        {
            return ClientFFI.GetServerTime(clientId, out time);
        }
    }
}