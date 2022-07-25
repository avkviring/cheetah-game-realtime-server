using System.Collections.Generic;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate.Abstract
{
    /// <summary>
    /// Маршуртизация события по типу шаблона (только для событий жизненного цикла объекта)
    /// </summary>
    public class AbstractRouterByTemplate<T> : Plugin
    {
        public delegate void Listener(T value);

        protected readonly Dictionary<ushort, Listeners> listenersByTemplate = new();
        protected CheetahObjectsCreateInfo objectsCreateInfo;

        protected class Listeners
        {
            public event Listener Listener;

            public void Notify(T value)
            {
                Listener?.Invoke(value);
            }
        }

        public virtual void Init(CheetahClient client)
        {
            objectsCreateInfo = client.GetPlugin<CheetahObjectsCreateInfo>();
        }

        public void RegisterListener(ushort template, Listener listener)
        {
            if (!listenersByTemplate.TryGetValue(template, out var listeners))
            {
                listeners = new Listeners();
                listenersByTemplate.Add(template, listeners);
            }

            listeners.Listener += listener;
        }


        public void UnRegisterListener(ushort fieldId, Listener listener)
        {
            if (listenersByTemplate.TryGetValue(fieldId, out var listeners))
            {
                listeners.Listener -= listener;
            }
        }
    }
}