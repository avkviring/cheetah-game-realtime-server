using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal
{
    public static class ProtocolExtensions
    {
        public static void SetStructure<T>(this CheetahClient client, ref CheetahBuffer buffer,
            in CheetahObjectId objectId, ushort fieldId, in T item)
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(StructureFFI.Set(client.Id, in objectId, fieldId, ref buffer));
        }

        public static void CompareAndSetStructure<T>(this CheetahClient client, ref CheetahBuffer buffer,
            in CheetahObjectId objectId, ushort fieldId, in T current, in T newval)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = client.CodecRegistry.GetCodec<T>();
            codec.Encode(in current, ref buffer);
            codec.Encode(in newval, ref newBuffer);

            ResultChecker.Check(StructureFFI.CompareAndSet(
                client.Id,
                in objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                false,
                ref resetBuffer
            ));
        }

        public static void CompareAndSetStructureWithReset<T>(this CheetahClient client, ref CheetahBuffer buffer,
            in CheetahObjectId objectId, ushort fieldId, in T current, in T newval, in T reset)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = client.CodecRegistry.GetCodec<T>();
            codec.Encode(in current, ref buffer);
            codec.Encode(in newval, ref newBuffer);
            codec.Encode(in reset, ref resetBuffer);

            ResultChecker.Check(StructureFFI.CompareAndSet(
                client.Id,
                in objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                true,
                ref resetBuffer
            ));
        }

        public static void SendEvent<T>(this CheetahClient client, ref CheetahBuffer buffer,
            in CheetahObjectId objectId, ushort eventId, in T item)
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(EventFFI.Send(client.Id, in objectId, eventId, ref buffer));
        }

        public static void SendEvent<T>(this CheetahClient client, ref CheetahBuffer buffer,
            in CheetahObjectId objectId, ushort eventId, uint targetUser, in T item)
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(EventFFI.Send(client.Id, (ushort)targetUser, in objectId, eventId, ref buffer));
        }

        public static void SetLong(this CheetahClient client, in CheetahObjectId objectId, ushort fieldId, long value)
        {
            ResultChecker.Check(LongFFI.Set(client.Id, in objectId, fieldId, value));
        }

        public static void IncrementLong(this CheetahClient client, in CheetahObjectId objectId, ushort fieldId,
            long increment)
        {
            ResultChecker.Check(LongFFI.Increment(client.Id, in objectId, fieldId, increment));
        }

        public static void CompareAndSetLong(this CheetahClient client, in CheetahObjectId objectId, ushort fieldId,
            long currentValue, long newValue)
        {
            ResultChecker.Check(LongFFI.CompareAndSet(client.Id, in objectId, fieldId, currentValue, newValue, false, 0));
        }

        public static void CompareAndSetLongWithReset(this CheetahClient client, in CheetahObjectId objectId,
            ushort fieldId, long currentValue, long newValue, long resetValue)
        {
            ResultChecker.Check(LongFFI.CompareAndSet(client.Id, in objectId, fieldId, currentValue, newValue, true, resetValue));
        }

        public static void SetDouble(this CheetahClient client, in CheetahObjectId objectId, ushort fieldId,
            double value)
        {
            ResultChecker.Check(DoubleFFI.Set(client.Id, in objectId, fieldId, value));
        }

        public static void DeleteField(this CheetahClient client, in CheetahObjectId objectId, ushort fieldId,
            FieldType fieldType)
        {
            ResultChecker.Check(FieldFFI.Delete(client.Id, in objectId, fieldId, fieldType));
        }

        public static void IncrementDouble(this CheetahClient client, in CheetahObjectId objectId, ushort fieldId,
            double increment)
        {
            ResultChecker.Check(DoubleFFI.Increment(client.Id, in objectId, fieldId, increment));
        }

        public static void Delete(this CheetahClient client, in CheetahObjectId objectId)
        {
            ResultChecker.Check(ObjectFFI.Delete(client.Id, in objectId));
        }
    }
}