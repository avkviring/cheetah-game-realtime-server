using System.Collections.Generic;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId
{
    /// <summary>
    /// Маршрутизация события по типу поля и по игровому объекту
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class AbstractRouterByObject<T> : Plugin
    {
        public delegate void Listener(ushort commandCreator, CheetahObject cheetahObject, bool created, ref T data);

        protected struct Key
        {
            internal CheetahObjectId id;
            internal ushort fieldId;

            public bool Equals(Key other)
            {
                return id.Equals(other.id) && fieldId == other.fieldId;
            }

            public override bool Equals(object obj)
            {
                return obj is Key other && Equals(other);
            }

            public override int GetHashCode()
            {
                unchecked
                {
                    return (id.GetHashCode() * 397) ^ fieldId.GetHashCode();
                }
            }
        }

        protected readonly Dictionary<Key, Listeners> listenersByFieldId = new Dictionary<Key, Listeners>();

        private CheetahObjectsCreateInfo createInfo;

        protected class Listeners
        {
            public event Listener OnEventReceive;

            public void Notify(ushort commandCreator, CheetahObject cheetahObject, bool created, ref T data)
            {
                OnEventReceive?.Invoke(commandCreator, cheetahObject, created, ref data);
            }
        }

        public virtual void Init(CheetahClient client)
        {
            createInfo = client.GetPlugin<CheetahObjectsCreateInfo>();
        }

        public void RegisterListener(in CheetahObjectId id, ushort fieldId, Listener listener)
        {
            var key = CreateKey(id, fieldId);
            if (!listenersByFieldId.TryGetValue(key, out var listeners))
            {
                listeners = new Listeners();
                listenersByFieldId.Add(key, listeners);
            }

            listeners.OnEventReceive += listener;
        }

        private static Key CreateKey(CheetahObjectId id, ushort fieldId)
        {
            return new Key
            {
                fieldId = fieldId,
                id = id
            };
        }


        public void UnRegisterListener(in CheetahObjectId id, ushort fieldId, Listener listener)
        {
            if (listenersByFieldId.TryGetValue(CreateKey(id, fieldId), out var listeners))
            {
                listeners.OnEventReceive -= listener;
            }
        }

        protected void Notify(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref T data)
        {
            if (listenersByFieldId.TryGetValue(CreateKey(objectId, fieldId), out var listeners))
            {
                listeners.Notify(commandCreator, createInfo.GetObject(in objectId), createInfo.IsCreated(in objectId), ref data);
            }
        }
    }
}