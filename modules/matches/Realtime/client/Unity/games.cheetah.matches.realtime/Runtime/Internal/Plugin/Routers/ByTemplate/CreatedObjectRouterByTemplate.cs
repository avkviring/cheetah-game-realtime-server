using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate.Abstract;
using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate
{
    /// <summary>
    /// Маршрутизация событий окончания создания объекта с фильтрацией по шаблону
    /// </summary>
    public class CreatedObjectRouterByTemplate : AbstractObjectEventRouterByTemplate
    {
        private ObjectCommandRouter objectCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectCreatedListener += OnObjectCreated;
        }

        private void OnObjectCreated(ref CheetahObjectId objectId)
        {
            Notify(ref objectId);
        }
    }
}