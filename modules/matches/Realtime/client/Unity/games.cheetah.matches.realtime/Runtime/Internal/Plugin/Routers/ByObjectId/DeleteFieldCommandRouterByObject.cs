using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId
{
    public class DeleteFieldCommandRouterByObjec : AbstractRouterByObject<FieldType>
    {
        private DeleteFieldCommandRouter router;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            router = client.GetPlugin<DeleteFieldCommandRouter>();
            router.DeleteListener += OnDelete;
        }

        private void OnDelete(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, FieldType fieldType)
        {
            Notify(commandCreator, ref objectId, fieldId, ref fieldType);
        }
    }
}