using Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate.Abstract;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate
{
    public class DeleteFieldRouterByTemplate : AbstractRouterByTemplate<DeletedField>
    {
        private DeleteFieldCommandRouter router;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            router = client.GetPlugin<DeleteFieldCommandRouter>();
            router.DeleteListener += OnFieldDelete;
        }

        private void OnFieldDelete(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, FieldType fieldType)
        {
            var cheetahObject = objectsCreateInfo.GetObject(in objectId);
            if (listenersByTemplate.TryGetValue(cheetahObject.Template, out var listeners))
            {
                listeners.Notify(new DeletedField
                {
                    cheetahObject = cheetahObject,
                    fieldId = fieldId,
                    fieldType = fieldType,
                });
            }
        }
    }

    public struct DeletedField
    {
        public CheetahObject cheetahObject;
        public ushort fieldId;
        public FieldType fieldType;
    }
}