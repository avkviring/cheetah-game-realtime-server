using System.Collections.Generic;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.ByField
{
    /// <summary>
    /// Маршрутизация события по типу поля
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class AbstractRouterByField<T> : Plugin
    {
        public delegate void Listener(ushort commandCreator, CheetahObject cheetahObject, bool created, ref T data);

        protected readonly Dictionary<ushort, Listeners> listenersByFieldId = new();
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

        public void RegisterListener(ushort fieldId, Listener listener)
        {
            if (!listenersByFieldId.TryGetValue(fieldId, out var listeners))
            {
                listeners = new Listeners();
                listenersByFieldId.Add(fieldId, listeners);
            }

            listeners.OnEventReceive += listener;
        }


        public void UnRegisterListener(ushort fieldId, Listener listener)
        {
            if (listenersByFieldId.TryGetValue(fieldId, out var listeners))
            {
                listeners.OnEventReceive -= listener;
            }
        }

        protected void Notify(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref T data)
        {
            if (listenersByFieldId.TryGetValue(fieldId, out var listeners))
            {
                listeners.Notify(commandCreator, createInfo.GetObject(ref objectId), createInfo.IsCreated(ref objectId), ref data);
            }
        }
    }
}