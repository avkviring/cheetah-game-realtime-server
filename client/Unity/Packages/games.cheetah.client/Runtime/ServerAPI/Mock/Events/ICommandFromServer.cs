namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public interface ICommandFromServer
    {
        public void Apply(CheetahClientMock cheetahClientMock);
    }
}