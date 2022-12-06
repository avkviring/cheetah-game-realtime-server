using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByField
{
    public class StructCommandRouterByField : AbstractRouterByField<CheetahBuffer>
    {
        private StructCommandRouter structCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            structCommandRouter = client.GetPlugin<StructCommandRouter>();
            structCommandRouter.ChangeListener += OnChange;
        }


        private void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            Notify(commandCreator, in objectId, fieldId, ref data);
        }
    }
}