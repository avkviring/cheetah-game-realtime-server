using System;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client
{
    public class Writer
    {
        private readonly IServerAPI serverAPI;
        private readonly CodecRegistry codecRegistry;
        private readonly ushort clientId;

        private CheetahBuffer buffer;

        public Writer(IServerAPI serverAPI, CodecRegistry codecRegistry, ushort clientId)
        {
            this.serverAPI = serverAPI;
            this.codecRegistry = codecRegistry;
            this.clientId = clientId;
        }

        public void SetLong(in CheetahObjectId objectId, ushort fieldId, long value)
        {
            ResultChecker.Check(serverAPI.Long.Set(clientId, in objectId, fieldId, value));
        }

        public void SetDouble(in CheetahObjectId objectId, ushort fieldId, double value)
        {
            ResultChecker.Check(serverAPI.Double.Set(clientId, in objectId, fieldId, value));
        }

        public void SetStructure<T>(in CheetahObjectId objectId, ushort fieldId, in T value) where T : struct
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in value, ref buffer);
            ResultChecker.Check(serverAPI.Structure.Set(clientId, in objectId, fieldId, ref buffer));
        }


        public void CompareAndSet<T>(in CheetahObjectId objectId, ushort fieldId, in T current, in T newValue) where T : unmanaged
        {
            DoCompareAndSet(in objectId, fieldId, in current, in newValue, false);
        }

        public void CompareAndSetWithReset<T>(in CheetahObjectId objectId, ushort fieldId, in T current, in T newValue, in T resetValue)
            where T : unmanaged
        {
            DoCompareAndSet(in objectId, fieldId, in current, in newValue, true, resetValue);
        }

        private void DoCompareAndSet<T>(in CheetahObjectId objectId, ushort fieldId, in T current, in T newValue, bool hasReset,
            T resetValue = default) where T : unmanaged
        {
            switch (current)
            {
                case long longCurrentValue:
                    if (newValue is long longNewValue && resetValue is long longResetValue)
                    {
                        ResultChecker.Check(serverAPI.Long.CompareAndSet(clientId, in objectId, fieldId, longCurrentValue, longNewValue, hasReset,
                            longResetValue));
                    }
                    else
                    {
                        throw new Exception("newValue ist not long");
                    }

                    break;
                default:
                    buffer.Clear();
                    var newBuffer = new CheetahBuffer();
                    var resetBuffer = new CheetahBuffer();
                    var codec = codecRegistry.GetCodec<T>();
                    codec.Encode(in current, ref buffer);
                    codec.Encode(in newValue, ref newBuffer);
                    if (hasReset)
                    {
                        codec.Encode(in resetValue, ref resetBuffer);
                    }

                    ResultChecker.Check(serverAPI.Structure.CompareAndSet(
                        clientId,
                        in objectId,
                        fieldId,
                        ref buffer,
                        ref newBuffer,
                        hasReset,
                        ref resetBuffer
                    ));
                    break;
            }
        }


        public void Increment<T>(in CheetahObjectId objectId, ushort fieldId, T increment) where T : unmanaged
        {
            switch (increment)
            {
                case long longIncrement:
                    ResultChecker.Check(serverAPI.Long.Increment(clientId, in objectId, fieldId, longIncrement));
                    break;
                case double doubleIncrement:
                    ResultChecker.Check(serverAPI.Double.Increment(clientId, in objectId, fieldId, doubleIncrement));
                    break;
                default:
                    throw new NotSupportedException("Increment is called with a unsupported type");
            }
        }


        public void SendEvent<T>(in CheetahObjectId objectId, ushort eventId, in T item)
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(serverAPI.Event.Send(clientId, in objectId, eventId, ref buffer));
        }

        public void SendEvent<T>(in CheetahObjectId objectId, ushort eventId, uint targetUser, in T item)
        {
            buffer.Clear();
            codecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(serverAPI.Event.Send(clientId, (ushort)targetUser, in objectId, eventId, ref buffer));
        }

        public void DeleteField(in CheetahObjectId objectId, FieldType fieldType, ushort fieldId)
        {
            ResultChecker.Check(serverAPI.Field.Delete(clientId, in objectId, fieldId, fieldType));
        }


        public void Delete(in CheetahObjectId objectId)
        {
            ResultChecker.Check(serverAPI.Object.Delete(clientId, in objectId));
        }
    }
}