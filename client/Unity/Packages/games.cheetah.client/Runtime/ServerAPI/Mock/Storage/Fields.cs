using System;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public class Fields : IFieldServerAPI
    {
        internal IFieldServerAPI.Listener listener;

        private Doubles doubles;
        private Longs longs;
        private Structures structures;
        

        
        public Fields(Longs longs, Doubles doubles, Structures structures)
        {
            this.longs = longs;
            this.doubles = doubles;
            this.structures = structures;
        }
        
        public byte SetListener(ushort clientId, IFieldServerAPI.Listener listener)
        {
            this.listener = listener;
            return 0;
        }
        
        

        public byte Delete(ushort clientId, in CheetahObjectId objectId, ushort fieldId, FieldType fieldType)
        {
            switch (fieldType)
            {
                case FieldType.Long:
                    longs.DeleteField(objectId, fieldId);
                    break;
                case FieldType.Double:
                    doubles.DeleteField(objectId, fieldId);
                    break;
                case FieldType.Structure:
                    structures.DeleteField(objectId, fieldId);
                    break;
                case FieldType.Event:
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(fieldType), fieldType, null);
            }

            return 0;
        }
    }
}