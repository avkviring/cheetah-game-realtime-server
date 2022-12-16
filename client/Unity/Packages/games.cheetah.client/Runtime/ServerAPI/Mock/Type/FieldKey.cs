using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Type
{
    public struct FieldKey<FID> : IEquatable<FieldKey<FID>> where FID : FieldId
    {
        private readonly CheetahObjectId objectId;
        private readonly FID fieldId;

        public FieldKey(CheetahObjectId objectId, FID fieldId)
        {
            this.objectId = objectId;
            this.fieldId = fieldId;
        }

        public bool Equals(FieldKey<FID> other)
        {
            return objectId.Equals(other.objectId) && EqualityComparer<FID>.Default.Equals(fieldId, other.fieldId);
        }

        public override bool Equals(object obj)
        {
            return obj is FieldKey<FID> other && Equals(other);
        }

        public override int GetHashCode()
        {
            unchecked
            {
                return (objectId.GetHashCode() * 397) ^ EqualityComparer<FID>.Default.GetHashCode(fieldId);
            }
        }
    }
}