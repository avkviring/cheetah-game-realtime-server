using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate.Abstract;
using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate
{
    /// <summary>
    /// Маршрутизация событий создания объекта с фильтрацией по шаблону
    /// </summary>
    public class CreatingObjectRouterByTemplate : AbstractObjectEventRouterByTemplate
    {
        private ObjectCommandRouter objectCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectCreatingListener += OnObjectCreating;
        }

        private void OnObjectCreating(in CheetahObjectId objectId, ushort template)
        {
            Notify(in objectId);
        }
    }
}