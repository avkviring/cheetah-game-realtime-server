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

        public object DecodeStructure(ushort structureId, ref CheetahBuffer cheetahBuffer)
        {
            return getCodec(structureId, structures).Decode(ref cheetahBuffer);
        }

        public object DecodeEvent(ushort eventId, ref CheetahBuffer cheetahBuffer)
        {
            return getCodec(eventId, events).Decode(ref cheetahBuffer);
        }

        public void EncodeStructure(ushort structureId, object structure, ref CheetahBuffer cheetahBuffer)
        {
            getCodec(structureId, structures).Encode(structure, ref cheetahBuffer);
        }


        public void EncodeEvent(ushort eventId, object structure, ref CheetahBuffer cheetahBuffer)
        {
            getCodec(eventId, events).Encode(structure, ref cheetahBuffer);
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
        object Decode(ref CheetahBuffer cheetahBuffer);

        void Encode(object value, ref CheetahBuffer cheetahBuffer);
    }
}