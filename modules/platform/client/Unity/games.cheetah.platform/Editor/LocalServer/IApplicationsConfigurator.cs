using Cheetah.Platform.Editor.LocalServer.Runner;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer
{
    /// <summary>
    /// UI для конфигурирования одного или нескольких ServerApplication
    /// </summary>
    public interface IApplicationsConfigurator
    {
        /// <summary>
        /// Заголовок элемента
        /// </summary>
        string Title { get; }

        /// <summary>
        /// UI представление элемента
        /// </summary>
        /// <returns></returns>
        VisualElement CreateUI();

        /// <summary>
        /// Обновление статуса запуска
        /// </summary>
        void OnUpdateStatus(Status status);


        /// <summary>
        /// Вес для сортировки, используется совместно с Title.
        /// Чем больше - тем выше элемент.
        /// </summary>
        int Order { get; }
    }
}