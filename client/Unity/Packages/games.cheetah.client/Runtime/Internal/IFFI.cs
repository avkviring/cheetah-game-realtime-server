using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client.Internal
{
    public interface IFFI
    {
        byte CreateClient(
            string serverAddress,
            ushort memberId,
            ulong roomId,
            ref NetworkBuffer userPrivateKey,
            ulong startFrameId,
            out ushort clientId
        );

        byte GetConnectionStatus(ushort clientId, out ConnectionStatus status);
        byte GetStatistics(ushort clientId, out Statistics clientStatistics);
        unsafe byte Receive(ushort clientId, S2CCommand* commands, ref ushort count);
        byte DestroyClient(ushort clientId);
        byte AttachToRoom(ushort clientId);
        byte DetachFromRoom(ushort clientId);
        byte SetChannelType(ushort clientId, NetworkChannelType networkChannelType, byte group);
        byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion);
        byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs);
        byte ResetEmulation(ushort clientId);
        void GetLastErrorMsg(ref NetworkBuffer buffer);
        byte GetServerTime(ushort clientId, out ulong time);

        byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Double fieldId, double value);
        byte Increment(ushort clientId, in NetworkObjectId objectId, FieldId.Double fieldId, double increment);

        byte Send(ushort clientId, in NetworkObjectId objectId, FieldId.Event fieldId, ref NetworkBuffer eventData);
        byte Send(ushort clientId, ushort targetUser, in NetworkObjectId objectId, FieldId.Event fieldId, ref NetworkBuffer eventData);

        byte DeleteField(ushort clientId, in NetworkObjectId objectId, FieldId fieldId);

        byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Long fieldId, long value);
        byte Increment(ushort clientId, in NetworkObjectId objectId, FieldId.Long fieldId, long increment);

        byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref NetworkObjectId objectId);
        byte CreatedObject(ushort clientId, in NetworkObjectId objectId, bool roomOwner, ref NetworkBuffer singletonKey);
        byte DeleteObject(ushort clientId, in NetworkObjectId objectId);

        byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Structure fieldId, ref NetworkBuffer value);
    }
}