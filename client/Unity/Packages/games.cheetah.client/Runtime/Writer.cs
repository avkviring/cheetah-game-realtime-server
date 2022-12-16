using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client
{
    public class Writer
    {
        private readonly IServerAPI serverAPI;
        private readonly CodecRegistry codecRegistry;
        private readonly ushort clientId;

        private CheetahBuffer buffer;

        public Writer(IServerAPI serverAPI, CodecRegistry codecRegistry, ushort clientId)
        {
            this.serverAPI = serverAPI;
            this.codecRegistry = codecRegistry;
            this.clientId = clientId;
        }

        public void SetLong(in CheetahObjectId objectId, FieldId.Long fieldId, long value)
        {
            ResultChecker.Check(serverAPI.Long.Set(clientId, in objectId, fieldId, value));
        }

        public void SetDouble(in CheetahObjectId objectId, FieldId.Double fieldId, double value)
        {
            ResultChecker.Check(serverAPI.Double.Set(clientId, in objectId, fieldId, value));
        }

        public void SetStructure<T>(in CheetahObjectId objectId, FieldId.Structure fieldId, in T value) where T : struct
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in value, ref buffer);
            ResultChecker.Check(serverAPI.Structure.Set(clientId, in objectId, fieldId, ref buffer));
        }

        public void CompareAndSet(in CheetahObjectId objectId, FieldId.Long fieldId, long current, long newValue, bool hasReset = false,
            long resetValue = default)

        {
            ResultChecker.Check(serverAPI.Long.CompareAndSet(clientId, in objectId, fieldId, current, newValue, hasReset, resetValue));
        }


        public void CompareAndSet<T>(in CheetahObjectId objectId, FieldId.Structure fieldId, in T current, in T newValue) where T : unmanaged
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = codecRegistry.GetCodec<T>();
            codec.Encode(in current, ref buffer);
            codec.Encode(in newValue, ref newBuffer);

            ResultChecker.Check(serverAPI.Structure.CompareAndSet(
                clientId,
                in objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                false,
                ref resetBuffer
            ));
        }

        public void CompareAndSet<T>(in CheetahObjectId objectId, FieldId.Structure fieldId, in T current, in T newValue, in T resetValue)
            where T : unmanaged
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = codecRegistry.GetCodec<T>();
            codec.Encode(in current, ref buffer);
            codec.Encode(in newValue, ref newBuffer);
            codec.Encode(in resetValue, ref resetBuffer);

            ResultChecker.Check(serverAPI.Structure.CompareAndSet(
                clientId,
                in objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                true,
                ref resetBuffer
            ));
        }


        public void Increment(in CheetahObjectId objectId, FieldId.Long fieldId, long increment)
        {
            ResultChecker.Check(serverAPI.Long.Increment(clientId, in objectId, fieldId, increment));
        }

        public void Increment(in CheetahObjectId objectId, FieldId.Double fieldId, double increment)
        {
            ResultChecker.Check(serverAPI.Double.Increment(clientId, in objectId, fieldId, increment));
        }


        public void SendEvent<T>(in CheetahObjectId objectId, FieldId.Event eventId, in T item)
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(serverAPI.Event.Send(clientId, in objectId, eventId, ref buffer));
        }

        public void SendEvent<T>(in CheetahObjectId objectId, FieldId.Event eventId, uint targetUser, in T item)
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(serverAPI.Event.Send(clientId, (ushort)targetUser, in objectId, eventId, ref buffer));
        }

        public void DeleteField(in CheetahObjectId objectId, FieldId fieldId)
        {
            ResultChecker.Check(serverAPI.Field.Delete(clientId, in objectId, fieldId));
        }


        public void Delete(in CheetahObjectId objectId)
        {
            ResultChecker.Check(serverAPI.Object.Delete(clientId, in objectId));
        }
    }
}