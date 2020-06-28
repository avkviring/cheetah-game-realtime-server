using System.Collections.Generic;
using System.Text;
using CheetahRelay.Runtime.LowLevel.External;
using CheetahRelay.Runtime.LowLevel.Listener;

namespace CheetahRelay.Tests.Runtime
{
    public class ServerCommandListenerStub : IServerCommandListener
    {
        public readonly List<string> Commands = new List<string>();

        public void OnObjectUploaded(in RelayGameObjectId objectId, in Structures structures, in LongCounters longCounters,
            in DoubleCounters floatCounters)
        {
            var builder = new StringBuilder();
            builder.Append("OnObjectUploaded[" + objectId.id + "]");
            var items = new List<string>();

            for (var i = 0; i < structures.count; i++)
            {
                unsafe
                {
                    var bytesCount = structures.sizes[i];
                    var binaryToStringBuilder = new StringBuilder();
                    binaryToStringBuilder.Append(" s[" + structures.fields[i] + "] = " + bytesCount + "(");
                    for (var valueIndex = 0; valueIndex < bytesCount; valueIndex++)
                    {
                        binaryToStringBuilder.Append(structures.values[i * Command.BufferSize + valueIndex].ToString("x2"));
                    }

                    binaryToStringBuilder.Append(")");
                    items.Add(binaryToStringBuilder.ToString());
                }
            }

            for (var i = 0; i < longCounters.count; i++)
            {
                unsafe
                {
                    items.Add(" l[" + longCounters.fields[i] + "]=" + longCounters.values[i] + " ");
                }
            }

            for (var i = 0; i < floatCounters.count; i++)
            {
                unsafe
                {
                    items.Add(" f[" + floatCounters.fields[i] + "]=" + floatCounters.values[i] + " ");
                }
            }

            items.Sort();
            foreach (var item in items)
            {
                builder.Append(item);
            }

            Commands.Add(builder.ToString());
        }

        public void OnLongCounterUpdated(in RelayGameObjectId objectId, ushort counterId, long value)
        {
            Commands.Add("OnLongCounterUpdated[" + objectId.id + "] counterId = " + counterId + ", value = " + value);
        }

        public void OnFloatCounterUpdated(in RelayGameObjectId objectId, ushort counterId, double value)
        {
            Commands.Add("OnFloatCounterUpdated[" + objectId.id + "] counterId = " + counterId + ", value = " + value);
        }

        public void OnStructureUpdated(in RelayGameObjectId objectId, ushort structureId, in Bytes data)
        {
            Commands.Add("OnStructureUpdated[" + objectId.id + "] structure id = " + structureId + ", data = " + data);
        }

        public void OnEvent(in RelayGameObjectId objectId, ushort eventId, in Bytes data)
        {
            Commands.Add("OnEvent[" + objectId.id + "] event id = " + eventId + ", data = " + data);
        }

        public void OnObjectUnloaded(in RelayGameObjectId objectId)
        {
            Commands.Add("OnObjectUnloaded[" + objectId.id + "]");
        }
    }
}