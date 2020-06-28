using CheetahRelay.Runtime.LowLevel.External;

namespace CheetahRelay.Runtime.LowLevel.Listener
{
    /**
     * Подписка на серверные события
     */
    public interface IServerCommandListener
    {
        void OnObjectUploaded(
            in RelayGameObjectId objectId,
            in Structures structures,
            in LongCounters longCounters,
            in DoubleCounters floatCounters
        );

        void OnLongCounterUpdated(in RelayGameObjectId objectId, ushort counterId, long value);

        void OnFloatCounterUpdated(in RelayGameObjectId objectId, ushort counterId, double value);

        void OnStructureUpdated(in RelayGameObjectId objectId, ushort structureId, in Bytes data);

        void OnEvent(in RelayGameObjectId objectId, ushort eventId, in Bytes data);

        void OnObjectUnloaded(in RelayGameObjectId objectId);
    }
}