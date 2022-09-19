using System;
using Cheetah.Matches.Realtime.Internal;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime
{
    /// <summary>
    /// Сетевой игровой объект
    /// </summary>
    public struct CheetahObject : IEquatable<CheetahObject>
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
            Client.SetStructure(in ObjectId, ref buffer, fieldId, in item);
        }

        public void CompareAndSetStructure<T>(ushort fieldId, ref T current, ref T newval)
        {
            Client.CompareAndSetStructure(in ObjectId, ref buffer, fieldId, in current, in newval);
        }

        public void CompareAndSetStructureWithReset<T>(ushort fieldId, ref T current, ref T newval, ref T reset)
        {
            Client.CompareAndSetStructureWithReset(in ObjectId, ref buffer, fieldId, in current, in newval, in reset);
        }

        public void SendEvent<T>(ushort eventId, ref T item)
        {
            Client.SendEvent(in ObjectId, ref buffer, eventId, in item);
        }

        public void SendEvent<T>(ushort eventId, uint targetUser, ref T item)
        {
            Client.SendEvent(in ObjectId, ref buffer, eventId, targetUser, in item);
        }

        public void SetLong(ushort fieldId, long value)
        {
            Client.SetLong(in ObjectId, fieldId, value);
        }

        public void IncrementLong(ushort fieldId, long increment)
        {
            Client.IncrementLong(in ObjectId, fieldId, increment);
        }

        public void CompareAndSetLong(ushort fieldId, long currentValue, long newValue)
        {
            Client.CompareAndSetLong(in ObjectId, fieldId, currentValue, newValue);
        }

        public void CompareAndSetLongWithReset(ushort fieldId, long currentValue, long newValue, long resetValue)
        {
            Client.CompareAndSetLongWithReset(in ObjectId, fieldId, currentValue, newValue, resetValue);
        }

        public void SetDouble(ushort fieldId, double value)
        {
            Client.SetDouble(in ObjectId, fieldId, value);
        }

        public void DeleteField(ushort fieldId, FieldType fieldType)
        {
            Client.DeleteField(in ObjectId, fieldId, fieldType);
        }

        public void IncrementDouble(ushort fieldId, double increment)
        {
            Client.IncrementDouble(in ObjectId, fieldId, increment);
        }

        /// <summary>
        /// Удалить игровой объект на сервере
        /// </summary>
        public void Delete()
        {
            Client.Delete(in ObjectId);
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