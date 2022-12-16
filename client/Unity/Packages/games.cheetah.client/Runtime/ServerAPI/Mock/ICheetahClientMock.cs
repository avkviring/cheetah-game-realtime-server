using System.Collections.Generic;
using Games.Cheetah.Client.ServerAPI.Mock.Events;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock
{
    /**
     * Mock для тестирования сетевого кода без сервера
     */
    public interface ICheetahClientMock
    {
        /**
         * Отправить команду от имени сервера, она выполнится при следующем вызове CheetahClient.Update
         */
        void ScheduleCommandFromServer(ICommandFromServer command);

        T? Get<T>(CheetahObjectId objectId, FieldId.Structure fieldId) where T : struct;

        long? Get(CheetahObjectId objectId, FieldId.Long fieldId);

        double? Get(CheetahObjectId objectId, FieldId.Double fieldId);

        void SetMemberIdForNewCheetahObject(ushort memberId);
        long GetCreatedObjectsCount();
        void Clear();
        IEnumerable<CheetahObjectId> GetCreatedObjects();
    }
}