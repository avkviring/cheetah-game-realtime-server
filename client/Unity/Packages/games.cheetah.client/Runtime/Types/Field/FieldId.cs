using System;

namespace Games.Cheetah.Client.Types.Field
{
    /**
     * Типизированный идентифкатор поля
     */
    public abstract class FieldId : IEquatable<FieldId>
    {
        public ushort Id { get; }
        public FieldType Type { get; }

        public FieldId(ushort id, FieldType type)
        {
            Id = id;
            Type = type;
        }


        public override string ToString()
        {
            return $"{nameof(Id)}: {Id}, {nameof(Type)}: {Type}";
        }

        public bool Equals(FieldId other)
        {
            if (ReferenceEquals(null, other)) return false;
            if (ReferenceEquals(this, other)) return true;
            return Id == other.Id && Type == other.Type;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            if (obj.GetType() != this.GetType()) return false;
            return Equals((FieldId)obj);
        }

        public override int GetHashCode()
        {
            unchecked
            {
                return (Id.GetHashCode() * 397) ^ (int)Type;
            }
        }

        public class Long : FieldId
        {
            public Long(ushort id) : base(id, FieldType.Long)
            {
            }
        }

        public class Double : FieldId
        {
            public Double(ushort id) : base(id, FieldType.Double)
            {
            }
        }

        public class Structure : FieldId
        {
            public Structure(ushort id) : base(id, FieldType.Structure)
            {
            }
        }

        public class Items : FieldId
        {
            public Items(ushort id) : base(id, FieldType.Items)
            {
            }
        }

        public class Event : FieldId
        {
            public Event(ushort id) : base(id, FieldType.Event)
            {
            }
        }
    }
}