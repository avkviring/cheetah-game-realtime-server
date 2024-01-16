using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client.Internal.FFI
{
    public class FFIImpl : IFFI
    {
        public byte CreateClient(ulong connectionId, string serverAddress, ushort memberId, ulong roomId, ref NetworkBuffer userPrivateKey,
            ulong disconnectTimeInSec,
            out ushort clientId)
        {
            return FFIMethods.CreateClient(connectionId, serverAddress, memberId, roomId, ref userPrivateKey, disconnectTimeInSec, out clientId);
        }

        public byte GetConnectionStatus(ushort clientId, out ConnectionStatus status)
        {
            return FFIMethods.GetConnectionStatus(clientId, out status);
        }

        public byte GetStatistics(ushort clientId, out Statistics clientStatistics)
        {
            return FFIMethods.GetStatistics(clientId, out clientStatistics);
        }


        public unsafe byte Receive(ushort clientId, S2CCommand* commands, ref ushort count)
        {
            return FFIMethods.Receive(clientId, commands, ref count);
        }

        public byte DestroyClient(ushort clientId)
        {
            return FFIMethods.DestroyClient(clientId);
        }

        public byte AttachToRoom(ushort clientId)
        {
            return FFIMethods.AttachToRoom(clientId);
        }

        public byte DetachFromRoom(ushort clientId)
        {
            return FFIMethods.DetachFromRoom(clientId);
        }

        public byte SetChannelType(ushort clientId, ReliabilityGuarantees reliabilityGuarantees, byte group)
        {
            return FFIMethods.SetChannelType(clientId, reliabilityGuarantees, group);
        }

        public byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion)
        {
            return FFIMethods.SetRttEmulation(clientId, rttInMs, rttDispersion);
        }

        public byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs)
        {
            return FFIMethods.SetDropEmulation(clientId, dropProbability, dropTimeInMs);
        }

        public byte ResetEmulation(ushort clientId)
        {
            return FFIMethods.ResetEmulation(clientId);
        }

        public void GetLastErrorMsg(ref NetworkBuffer buffer)
        {
            FFIMethods.GetLastErrorMsg(ref buffer);
        }

        public byte GetServerTime(ushort clientId, out ulong time)
        {
            return FFIMethods.GetServerTime(clientId, out time);
        }

        public byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Double fieldId, double value)
        {
            return FFIMethods.Set(clientId, in objectId, fieldId.Id, value);
        }

        public byte Increment(ushort clientId, in NetworkObjectId objectId, FieldId.Double fieldId, double increment)
        {
            return FFIMethods.Increment(clientId, in objectId, fieldId.Id, increment);
        }

        public byte Send(ushort clientId, in NetworkObjectId objectId, FieldId.Event fieldId, ref NetworkBuffer eventData)
        {
            return FFIMethods.Send(clientId, in objectId, fieldId.Id, ref eventData);
        }

        public byte Send(ushort clientId, ushort targetUser, in NetworkObjectId objectId, FieldId.Event fieldId, ref NetworkBuffer eventData)
        {
            return FFIMethods.Send(clientId, targetUser, in objectId, fieldId.Id, ref eventData);
        }

        public byte DeleteField(ushort clientId, in NetworkObjectId objectId, FieldId fieldId)
        {
            return FFIMethods.DeleteField(clientId, in objectId, fieldId.Id, fieldId.Type);
        }

        public byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Long fieldId, long value)
        {
            return FFIMethods.Set(clientId, in objectId, fieldId.Id, value);
        }

        public byte Increment(ushort clientId, in NetworkObjectId objectId, FieldId.Long fieldId, long increment)
        {
            return FFIMethods.Increment(clientId, in objectId, fieldId.Id, increment);
        }

        public byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref NetworkObjectId objectId)
        {
            return FFIMethods.CreateObject(clientId, template, accessGroup, ref objectId);
        }

        public byte CreatedObject(ushort clientId, in NetworkObjectId objectId, bool roomOwner, ref NetworkBuffer singletonKey)
        {
            return FFIMethods.CreatedObject(clientId, in objectId, roomOwner, ref singletonKey);
        }

        public byte DeleteObject(ushort clientId, in NetworkObjectId objectId)
        {
            return FFIMethods.DeleteObject(clientId, in objectId);
        }

        public byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Structure fieldId, ref NetworkBuffer value)
        {
            return FFIMethods.Set(clientId, in objectId, fieldId.Id, ref value);
        }

        public byte AddItem(ushort clientId, in NetworkObjectId objectId, FieldId.Items fieldId, ref NetworkBuffer buffer)
        {
            return FFIMethods.AddItem(clientId, in objectId, fieldId.Id, ref buffer);
        }
    }
}