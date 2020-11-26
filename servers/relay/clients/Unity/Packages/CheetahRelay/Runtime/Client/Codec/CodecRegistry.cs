using System;
using System.Collections.Generic;

namespace CheetahRelay.Runtime.Client.Codec
{
    /**
     * Сериализация структур/событий
     */
    public class CodecRegistry
    {
        private Dictionary<ushort, Codec> structures = new Dictionary<ushort, Codec>();
        private Dictionary<ushort, Codec> events = new Dictionary<ushort, Codec>();


        public void RegisterStructure(ushort structureId, Codec codec)
        {
            structures[structureId] = codec;
        }

        public void RegisterEvent(ushort eventId, Codec codec)
        {
            events[eventId] = codec;
        }

        public object DecodeStructure(ushort structureId, ref RelayBuffer relayBuffer)
        {
            return getCodec(structureId, structures).Decode(ref relayBuffer);
        }

        public object DecodeEvent(ushort eventId, ref RelayBuffer relayBuffer)
        {
            return getCodec(eventId, events).Decode(ref relayBuffer);
        }

        public void EncodeStructure(ushort structureId, object structure, ref RelayBuffer relayBuffer)
        {
            getCodec(structureId, structures).Encode(structure, ref relayBuffer);
        }


        public void EncodeEvent(ushort eventId, object structure, ref RelayBuffer relayBuffer)
        {
            getCodec(eventId, events).Encode(structure, ref relayBuffer);
        }


        private Codec getCodec(ushort fieldId, Dictionary<ushort, Codec> codecs)
        {
            try
            {
                return codecs[fieldId];
            }
            catch (KeyNotFoundException e)
            {
                throw new ArgumentNullException("Codec not found, fieldId = " + fieldId);
            }
        }
    }


    public interface Codec
    {
        object Decode(ref RelayBuffer relayBuffer);

        void Encode(object value, ref RelayBuffer relayBuffer);
    }
}