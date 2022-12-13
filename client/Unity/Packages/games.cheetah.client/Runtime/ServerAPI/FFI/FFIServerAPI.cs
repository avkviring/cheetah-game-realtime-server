namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class FFIServerAPI : IServerAPI
    {
        public IClientServerAPI Client { get; } = new ClientServerAPI();
        public IDoubleServerAPI Double { get; } = new DoubleServerAPI();
        public IEventServerAPI Event { get; } = new EventServerAPI();
        public IFieldServerAPI Field { get; } = new FieldServerAPI();
        public ILongServerAPI Long { get; } = new LongServerAPI();
        public IObjectServerAPI Object { get; } = new ObjectServerAPI();
        public IStructureServerAPI Structure { get; } = new StructureServerAPI();
    }
}