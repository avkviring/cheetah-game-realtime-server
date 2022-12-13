namespace Games.Cheetah.Client.ServerAPI
{
    public interface IServerAPI
    {
        IClientServerAPI Client { get; }
        IDoubleServerAPI Double { get; }
        IEventServerAPI Event { get; }
        IFieldServerAPI Field { get; }
        ILongServerAPI Long { get; }
        IObjectServerAPI Object { get; }
        IStructureServerAPI Structure { get; }
    }
}