using System.Collections.Generic;
using Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal
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

        private void OnObjectCreating(ref CheetahObjectId objectId, ushort template)
        {
            templates.Add(objectId, template);
        }

        private void OnObjectCreated(ref CheetahObjectId objectId)
        {
            created.Add(objectId);
        }

        private void OnDeleted(ref CheetahObjectId objectId)
        {
            templates.Remove(objectId);
            created.Remove(objectId);
        }

        public CheetahObject GetObject(ref CheetahObjectId objectId)
        {
            return new CheetahObject(objectId, templates[objectId], client);
        }

        public bool IsCreated(ref CheetahObjectId objectId)
        {
            return created.Contains(objectId);
        }

        public void OnLocalObjectCreating(ref CheetahObjectId objectId, ushort template)
        {
            OnObjectCreating(ref objectId, template);
        }

        public void OnLocalObjectCreate(ref CheetahObjectId objectId)
        {
            OnObjectCreated(ref objectId);
        }
    }
}