#nullable enable
using System.Collections.Generic;
using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client
{
    /**
     * Mock для тестов с участием сетевого клиента
     * - все команды ScheduleCommand* выполняются в клиенте после вызова Update
     * - все команды GetValue* выдают значения как из ScheduleCommand, так и на вызовы методов Set из NetworkClient
     */
    public interface INetworkClientMock
    {
        void ScheduleCommandFromServer(S2CCommands.CreateObject command);
        void ScheduleCommandFromServer(S2CCommands.CreatedObject command);
        void ScheduleCommandFromServer(S2CCommands.SetLong command);
        void ScheduleCommandFromServer(S2CCommands.SetDouble command);
        void ScheduleCommandFromServer(S2CCommands.DeleteObject command);

        void ScheduleCommandFromServer(S2CCommands.SetStructure command);


        long? GetFieldValue(NetworkObjectId id, FieldId.Long field);
        double? GetFieldValue(NetworkObjectId id, FieldId.Double field);
        T? GetFieldValue<T>(NetworkObjectId id, FieldId.Structure field) where T : struct;

        void SetMemberIdForNewNetworkObject(ushort memberId);

        IList<NetworkObjectId> GetCreatedObjects();
        long GetCreatedObjectsCount();
        void Clear();
    }
}