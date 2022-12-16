using System.Collections.Generic;
using Games.Cheetah.Client.ServerAPI.Mock.Type;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public abstract class AbstractStorage<T, FID> where FID : FieldId
    {
        protected readonly Dictionary<FieldKey<FID>, T> fields = new();

        public byte Set(ushort clientId, in CheetahObjectId objectId, FID fieldId, ref T data)
        {
            var key = new FieldKey<FID>(objectId, fieldId);
            fields[key] = data;
            return 0;
        }

        public bool TryGetFieldValue(CheetahObjectId objectId, FID fieldId, out T value)
        {
            var key = new FieldKey<FID>(objectId, fieldId);
            return fields.TryGetValue(key, out value);
        }


        public void Clear()
        {
            fields.Clear();
        }

        public void DeleteField(in CheetahObjectId objectId, FID fieldId)
        {
            fields.Remove(new FieldKey<FID>(objectId, fieldId));
        }
    }
}