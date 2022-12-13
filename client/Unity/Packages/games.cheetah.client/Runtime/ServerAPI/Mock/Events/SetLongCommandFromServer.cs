using Games.Cheetah.Client.Types;
using UnityEngine;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class SetLongCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;
        private readonly ushort fieldId;
        private readonly long value;

        public SetLongCommandFromServer(CheetahObjectId objectId, ushort fieldId, long value)
        {
            this.objectId = objectId;
            this.fieldId = fieldId;
            this.value = value;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            cheetahClientMock.longs.listener?.Invoke(0, in objectId, fieldId, value);
        }
    }
}