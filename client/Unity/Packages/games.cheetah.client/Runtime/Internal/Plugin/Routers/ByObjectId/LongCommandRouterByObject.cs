using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByObjectId
{
    public class LongCommandRouterByObject : AbstractRouterByObject<long>
    {
        private LongCommandRouter longCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            longCommandRouter = client.GetPlugin<LongCommandRouter>();
            longCommandRouter.ChangeListener += OnChange;
        }

        private void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, long value)
        {
            Notify(commandCreator, in objectId, fieldId, ref value);
        }
    }
}