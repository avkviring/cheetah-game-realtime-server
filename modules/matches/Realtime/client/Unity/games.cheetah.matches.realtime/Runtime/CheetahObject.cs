using Cheetah.Matches.Realtime.Internal;
using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime
{
    /// <summary>
    /// Сетевой игровой объект
    /// </summary>
    public struct CheetahObject
    {
        private static CheetahBuffer buffer;
        public CheetahObjectId ObjectId;
        public ushort Template;
        public readonly CheetahClient Client;

        public CheetahObject(CheetahObjectId objectId, ushort template, CheetahClient client)
        {
            ObjectId = objectId;
            Template = template;
            Client = client;
        }

        public void SetStructure<T>(ushort fieldId, ref T item)
        {
            buffer.Clear();
            Client.CodecRegistry.GetCodec<T>().Encode(ref item, ref buffer);
            ResultChecker.Check(StructureFFI.Set(Client.Id, ref ObjectId, fieldId, ref buffer));
        }

        public void CompareAndSetStructure<T>(ushort fieldId, ref T current, ref T newval)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = Client.CodecRegistry.GetCodec<T>();
            codec.Encode(ref current, ref buffer);
            codec.Encode(ref newval, ref newBuffer);

            ResultChecker.Check(StructureFFI.CompareAndSet(
                Client.Id,
                ref ObjectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                false,
                ref resetBuffer
            ));
        }

        public void CompareAndSetStructureWithReset<T>(ushort fieldId, ref T current, ref T newval, ref T reset)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = Client.CodecRegistry.GetCodec<T>();
            codec.Encode(ref current, ref buffer);
            codec.Encode(ref newval, ref newBuffer);
            codec.Encode(ref reset, ref resetBuffer);

            ResultChecker.Check(StructureFFI.CompareAndSet(
                Client.Id,
                ref ObjectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                true,
                ref resetBuffer
            ));
        }

        public void SendEvent<T>(ushort eventId, ref T item)
        {
            buffer.Clear();
            Client.CodecRegistry.GetCodec<T>().Encode(ref item, ref buffer);
            ResultChecker.Check(EventFFI.Send(Client.Id, ref ObjectId, eventId, ref buffer));
        }

        public void SendEvent<T>(ushort eventId, uint targetUser, ref T item)
        {
            buffer.Clear();
            Client.CodecRegistry.GetCodec<T>().Encode(ref item, ref buffer);
            ResultChecker.Check(EventFFI.Send(Client.Id, (ushort)targetUser, ref ObjectId, eventId, ref buffer));
        }

        public void SetLong(ushort fieldId, long value)
        {
            ResultChecker.Check(LongFFI.Set(Client.Id, ref ObjectId, fieldId, value));
        }

        public void IncrementLong(ushort fieldId, long increment)
        {
            ResultChecker.Check(LongFFI.Increment(Client.Id, ref ObjectId, fieldId, increment));
        }

        public void CompareAndSetLong(ushort fieldId, long currentValue, long newValue)
        {
            ResultChecker.Check(LongFFI.CompareAndSet(Client.Id, ref ObjectId, fieldId, currentValue, newValue, false, 0));
        }

        public void CompareAndSetLongWithReset(ushort fieldId, long currentValue, long newValue, long resetValue)
        {
            ResultChecker.Check(LongFFI.CompareAndSet(Client.Id, ref ObjectId, fieldId, currentValue, newValue, true, resetValue));
        }

        public void SetDouble(ushort fieldId, double value)
        {
            ResultChecker.Check(DoubleFFI.Set(Client.Id, ref ObjectId, fieldId, value));
        }

        public void DeleteField(ushort fieldId, FieldType fieldType)
        {
            ResultChecker.Check(FieldFFI.Delete(Client.Id, ref ObjectId, fieldId, fieldType));
        }

        public void IncrementDouble(ushort fieldId, double increment)
        {
            ResultChecker.Check(DoubleFFI.Increment(Client.Id, ref ObjectId, fieldId, increment));
        }

        /// <summary>
        /// Удалить игровой объект на сервере
        /// </summary>
        public void Delete()
        {
            ResultChecker.Check(ObjectFFI.Delete(Client.Id, ref ObjectId));
        }

        public override string ToString()
        {
            return $"{nameof(ObjectId)}: {ObjectId}, {nameof(Template)}: {Template}";
        }

        public bool Equals(CheetahObject other)
        {
            return ObjectId.Equals(other.ObjectId);
        }

        public override bool Equals(object obj)
        {
            return obj is CheetahObject other && Equals(other);
        }

        public override int GetHashCode()
        {
            return ObjectId.GetHashCode();
        }
    }
}