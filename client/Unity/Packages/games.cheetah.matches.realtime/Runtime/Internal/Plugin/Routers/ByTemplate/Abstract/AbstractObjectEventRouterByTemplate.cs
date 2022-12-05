using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByTemplate.Abstract
{
    public class AbstractObjectEventRouterByTemplate : AbstractRouterByTemplate<CheetahObject>
    {
        
        protected void Notify(in CheetahObjectId objectId)
        {
            var cheetahObject = objectsCreateInfo.GetObject(in objectId);
            if (listenersByTemplate.TryGetValue(cheetahObject.Template, out var listeners))
            {
                listeners.Notify(cheetahObject);
            }
        }
    }
}