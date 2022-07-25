using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByField
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


        private void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            Notify(commandCreator, ref objectId, fieldId, ref data);
        }
    }
}