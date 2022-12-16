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


        public byte Delete(ushort clientId, in CheetahObjectId objectId, FieldId fieldId)
        {
            switch (fieldId)
            {
                case FieldId.Long longFieldId:
                    longs.DeleteField(objectId, longFieldId);
                    break;
                case FieldId.Double doubleFieldId:
                    doubles.DeleteField(objectId, doubleFieldId);
                    break;
                case FieldId.Structure structureFieldId:
                    structures.DeleteField(objectId, structureFieldId);
                    break;
            }

            return 0;
        }
    }
}