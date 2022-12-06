using System.Collections.Generic;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin
{
    /// <summary>
    /// Сбор инормации о создаваемых объектох
    /// TODO перенести функционал в rust для снижения нагрузки на память
    /// </summary>
    public class ObjectConstructorCollector : global::Games.Cheetah.Client.Internal.Plugin.Plugin
    {
        public delegate void ObjectCreate(CheetahObjectConstructor cheetahObjectConstructor);


        private readonly Dictionary<CheetahObjectId, CheetahObjectConstructor> creatingObjects =
            new Dictionary<CheetahObjectId, CheetahObjectConstructor>();

        private readonly Dictionary<ushort, CreatedListeners> createdListeners = new Dictionary<ushort, CreatedListeners>();
        private CheetahObjectsCreateInfo cheetahObjectsCreateInfo;
        private CheetahClient client;

        private class CreatedListeners
        {
            public event ObjectCreate OnCreateListener;

            internal void OnCreate(CheetahObjectConstructor cheetahObjectConstructor)
            {
                OnCreateListener?.Invoke(cheetahObjectConstructor);
            }
        }


        public void Init(CheetahClient client)
        {
            this.client = client;

            var objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectCreatingListener += OnObjectCreating;
            objectCommandRouter.ObjectCreatedListener += OnObjectCreated;
            objectCommandRouter.ObjectDeleteListener += OnObjectDelete;

            cheetahObjectsCreateInfo = client.GetPlugin<CheetahObjectsCreateInfo>();
            client.GetPlugin<StructCommandRouter>().ChangeListener += OnStructChange;
            client.GetPlugin<LongCommandRouter>().ChangeListener += OnLongChange;
            client.GetPlugin<DoubleCommandRouter>().ChangeListener += OnDoubleChange;
        }

        private void OnObjectDelete(in CheetahObjectId objectId)
        {
            creatingObjects.Remove(objectId);
        }

        public void RegisterListener(ushort template, ObjectCreate listener)
        {
            if (!createdListeners.TryGetValue(template, out var listeners))
            {
                listeners = new CreatedListeners();
                createdListeners.Add(template, listeners);
            }

            listeners.OnCreateListener += listener;
        }

        public void UnRegisterListener(ushort template, ObjectCreate listener)
        {
            if (createdListeners.TryGetValue(template, out var listeners))
            {
                listeners.OnCreateListener -= listener;
            }
        }

        private void OnObjectCreating(in CheetahObjectId objectId, ushort template)
        {
            var cheetahObject = cheetahObjectsCreateInfo.GetObject(in objectId);
            creatingObjects.Add(objectId, new CheetahObjectConstructor(cheetahObject, client.CodecRegistry));
        }


        private void OnStructChange(ushort creator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            if (creatingObjects.TryGetValue(objectId, out var creatingObject))
            {
                creatingObject.structures.Add(fieldId, data);
            }
        }

        private void OnDoubleChange(ushort creator, in CheetahObjectId objectId, ushort fieldId, double value)
        {
            if (creatingObjects.TryGetValue(objectId, out var creatingObject))
            {
                creatingObject.doubles.Add(fieldId, value);
            }
        }

        private void OnLongChange(ushort creator, in CheetahObjectId objectId, ushort fieldId, long value)
        {
            if (creatingObjects.TryGetValue(objectId, out var creatingObject))
            {
                creatingObject.longs.Add(fieldId, value);
            }
        }


        /// <summary>
        /// Объект создан - вызываем подписчиков
        /// </summary>
        /// <param name="objectId"></param>
        private void OnObjectCreated(in CheetahObjectId objectId)
        {
            var cheetahObject = cheetahObjectsCreateInfo.GetObject(in objectId);
            if (createdListeners.TryGetValue(cheetahObject.Template, out var listeners))
            {
                if (creatingObjects.TryGetValue(objectId, out var createdObject))
                {
                    listeners.OnCreate(createdObject);
                }
            }

            creatingObjects.Remove(objectId);
        }
    }
}