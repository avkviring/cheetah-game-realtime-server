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
        void ScheduleCreateObjectCommand(NetworkObjectId objectId, ushort template, ulong accessGroup);
        void ScheduleCreatedObjectCommand(NetworkObjectId objectId);
        void ScheduleSetLongCommand(NetworkObjectId id, FieldId.Long field, long value);
        void ScheduleSetDoubleCommand(NetworkObjectId id, FieldId.Double field, float value);
        void ScheduleDeleteObjectCommand(NetworkObjectId objectId);
        void ScheduleSendEventCommand(NetworkObjectId id, FieldId.Event field, NetworkBuffer value);
        void ScheduleSetStructureCommand(NetworkObjectId id, FieldId.Structure field, NetworkBuffer value);
        void ScheduleAddItemCommand(NetworkObjectId id, FieldId.Items field, NetworkBuffer value);

        long? GetFieldValue(NetworkObjectId id, FieldId.Long field);
        double? GetFieldValue(NetworkObjectId id, FieldId.Double field);
        T? GetFieldValue<T>(NetworkObjectId id, FieldId.Structure field) where T : struct;
        IEnumerable<T?> GetEvents<T>(NetworkObjectId id, FieldId.Event field) where T : struct;

        void SetMemberIdForNewNetworkObject(ushort memberId);

        IList<NetworkObjectId> GetCreatedObjects();
        long GetCreatedObjectsCount();
        void Clear();
        void ClearEvents();
    }
}