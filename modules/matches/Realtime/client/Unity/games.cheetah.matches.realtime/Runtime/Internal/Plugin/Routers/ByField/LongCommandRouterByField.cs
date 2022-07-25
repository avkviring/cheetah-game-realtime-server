using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByField
{
    public class LongCommandRouterByField : AbstractRouterByField<long>
    {
        private LongCommandRouter doubleCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            doubleCommandRouter = client.GetPlugin<LongCommandRouter>();
            doubleCommandRouter.ChangeListener += OnChange;
        }

        private void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, long value)
        {
            Notify(commandCreator, ref objectId, fieldId, ref value);
        }
    }
}