using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate.Abstract;
using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate
{
    public class DeleteObjectRouterByTemplate : AbstractObjectEventRouterByTemplate
    {
        private ObjectCommandRouter objectCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectDeleteListener += OnObjectDelete;
        }

        private void OnObjectDelete(in CheetahObjectId objectId)
        {
            Notify(in objectId);
        }
    }
}