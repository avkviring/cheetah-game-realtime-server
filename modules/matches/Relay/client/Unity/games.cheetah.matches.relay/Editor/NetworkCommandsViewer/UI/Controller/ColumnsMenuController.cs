using UnityEditor.UIElements;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.UI.Controller
{
    /**
     * Меню включения/выключения столбцов
     */
    public class ColumnsMenuController
    {
        private readonly Columns columns;
        private readonly ToolbarMenu menu;

        public ColumnsMenuController(ToolbarMenu menu, Columns columns)
        {
            this.menu = menu;
            this.columns = columns;
            menu.variant = ToolbarMenu.Variant.Default;
            UpdateMenu();
        }

        private void UpdateMenu()
        {
            menu.menu.MenuItems().Clear();
            foreach (var column in columns.AllColumns)
            {
                if (columns.IsEnable(column))
                    menu.menu.AppendAction(column.header, OnColumnsChanged, e => DropdownMenuAction.Status.Checked, column);
                else
                    menu.menu.AppendAction(column.header, OnColumnsChanged, e => DropdownMenuAction.Status.Normal, column);
            }
        }

        private void OnColumnsChanged(DropdownMenuAction action)
        {
            columns.SetEnable(action.userData as Column, action.status == DropdownMenuAction.Status.Normal);
            UpdateMenu();
        }
    }
}