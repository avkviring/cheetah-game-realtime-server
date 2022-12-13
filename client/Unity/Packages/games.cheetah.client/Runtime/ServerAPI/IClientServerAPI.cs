using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface IClientServerAPI
    {
        byte CreateClient(
            string serverAddress,
            ushort memberId,
            ulong roomId,
            ref CheetahBuffer userPrivateKey,
            ulong startFrameId,
            out ushort clientId
        );

        byte GetConnectionStatus(ushort clientId, out CheetahClientConnectionStatus status);
        byte GetStatistics(ushort clientId, out CheetahClientStatistics clientStatistics);
        byte Receive(ushort clientId);
        byte DestroyClient(ushort clientId);
        byte AttachToRoom(ushort clientId);
        byte DetachFromRoom(ushort clientId);
        byte SetChannelType(ushort clientId, ChannelType channelType, byte group);
        byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion);
        byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs);
        byte ResetEmulation(ushort clientId);
        void GetLastErrorMsg(ref CheetahBuffer buffer);
        byte GetServerTime(ushort clientId, out ulong time);
    }
}