using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client
{
    public class Writer
    {
        private readonly IFFI ffi;
        private readonly CodecRegistry codecRegistry;
        private readonly ushort clientId;

        private NetworkBuffer buffer;

        public Writer(IFFI ffi, CodecRegistry codecRegistry, ushort clientId)
        {
            this.ffi = ffi;
            this.codecRegistry = codecRegistry;
            this.clientId = clientId;
        }

        public void SetLong(in NetworkObjectId objectId, FieldId.Long fieldId, long value)
        {
            ResultChecker.Check(ffi.Set(clientId, in objectId, fieldId, value));
        }

        public void SetDouble(in NetworkObjectId objectId, FieldId.Double fieldId, double value)
        {
            ResultChecker.Check(ffi.Set(clientId, in objectId, fieldId, value));
        }

        public void SetStructure<T>(in NetworkObjectId objectId, FieldId.Structure fieldId, in T value) where T : struct
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in value, ref buffer);
            ResultChecker.Check(ffi.Set(clientId, in objectId, fieldId, ref buffer));
        }
        
        public void AddItem<T>(in NetworkObjectId objectId, FieldId.Items fieldId, in T value) where T : struct
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in value, ref buffer);
            ResultChecker.Check(ffi.AddItem(clientId, in objectId, fieldId, ref buffer));
        }


        public void Increment(in NetworkObjectId objectId, FieldId.Long fieldId, long increment)
        {
            ResultChecker.Check(ffi.Increment(clientId, in objectId, fieldId, increment));
        }

        public void Increment(in NetworkObjectId objectId, FieldId.Double fieldId, double increment)
        {
            ResultChecker.Check(ffi.Increment(clientId, in objectId, fieldId, increment));
        }


        public void SendEvent<T>(in NetworkObjectId objectId, FieldId.Event eventId, in T item)
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(ffi.Send(clientId, in objectId, eventId, ref buffer));
        }

        public void SendEvent<T>(in NetworkObjectId objectId, FieldId.Event eventId, uint targetUser, in T item)
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(ffi.Send(clientId, (ushort)targetUser, in objectId, eventId, ref buffer));
        }

        public void DeleteField(in NetworkObjectId objectId, FieldId fieldId) 
        {
            ResultChecker.Check(ffi.DeleteField(clientId, in objectId, fieldId));
        }


        public void Delete(in NetworkObjectId objectId)
        {
            ResultChecker.Check(ffi.DeleteObject(clientId, in objectId));
        }
    }
}