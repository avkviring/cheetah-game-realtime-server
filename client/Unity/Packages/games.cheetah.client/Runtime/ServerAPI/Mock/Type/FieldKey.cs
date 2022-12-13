using System;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Type
{
    public struct FieldKey : IEquatable<FieldKey>
    {
        private readonly CheetahObjectId objectId;
        private readonly ushort fieldId;

        public FieldKey(CheetahObjectId objectId, ushort fieldId)
        {
            this.objectId = objectId;
            this.fieldId = fieldId;
        }

        public bool Equals(FieldKey other)
        {
            return objectId.Equals(other.objectId) && fieldId == other.fieldId;
        }

        public override bool Equals(object obj)
        {
            return obj is FieldKey other && Equals(other);
        }

        public override int GetHashCode()
        {
            unchecked
            {
                return (objectId.GetHashCode() * 397) ^ fieldId.GetHashCode();
            }
        }
    }
}