namespace Cheetah.Platform.Editor.LocalServer.Runner
{
    /// <summary>
    ///     Оповещение об измении прогресса запуска/останова контейнеров
    /// </summary>
    public interface IDockerProgressListener
    {
        void SetProgressTitle(string title);
        void SetProgress(int percent);
    }
}