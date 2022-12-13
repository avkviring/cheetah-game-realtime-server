using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal
{
    /// <summary>
    /// Информация о шаблоне и этапе создания игровых объектов
    /// </summary>
    public class CheetahObjectsCreateInfo : Plugin.Plugin
    {
        private readonly Dictionary<CheetahObjectId, ushort> templates = new Dictionary<CheetahObjectId, ushort>();
        private readonly HashSet<CheetahObjectId> created = new HashSet<CheetahObjectId>();
        private CheetahClient client;


        public void Init(CheetahClient client)
        {
            this.client = client;
            var router = client.GetPlugin<ObjectCommandRouter>();
            router.ObjectCreatingListener += OnObjectCreating;
            router.ObjectCreatedListener += OnObjectCreated;
            router.ObjectPostDeleteListener += OnDeleted;
        }

        private void OnObjectCreating(in CheetahObjectId objectId, ushort template)
        {
            templates.Add(objectId, template);
        }

        private void OnObjectCreated(in CheetahObjectId objectId)
        {
            created.Add(objectId);
        }

        private void OnDeleted(in CheetahObjectId objectId)
        {
            templates.Remove(objectId);
            created.Remove(objectId);
        }

        public CheetahObject GetObject(in CheetahObjectId objectId)
        {
            if (templates.TryGetValue(objectId, out var template))
            {
                return new CheetahObject(objectId, template);
            }

            throw new Exception("CheetahObject not created " + objectId);
        }

        public bool IsCreated(in CheetahObjectId objectId)
        {
            return created.Contains(objectId);
        }

        public void OnLocalObjectCreating(in CheetahObjectId objectId, ushort template)
        {
            OnObjectCreating(in objectId, template);
        }

        public void OnLocalObjectCreate(in CheetahObjectId objectId)
        {
            OnObjectCreated(in objectId);
        }
    }
}